package main

import (
	"fmt"
	"html/template"
	"io"
	"log"
	"net/http"
	"sync"

	"github.com/go-chi/chi"
	"github.com/go-chi/chi/middleware"
	"github.com/google/uuid"
)

type AppState struct {
	sync.Mutex
	Sessions map[uuid.UUID]int
}

type templates map[string]*template.Template

func main() {
	t := templates{
		"index":   template.Must(template.ParseFiles("templates/layout.html", "templates/index.html")),
		"session": template.Must(template.ParseFiles("templates/layout.html", "templates/session.html")),
	}

	state := AppState{
		Sessions: map[uuid.UUID]int{},
	}

	r := chi.NewRouter()
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)

	r.Get("/", func(w http.ResponseWriter, r *http.Request) {
		err := t["index"].Execute(w, nil)
		if err != nil {
			log.Print(err.Error())
			http.Error(w, "internal server error", 500)
		}
	})

	r.Post("/session/create", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			w.WriteHeader(http.StatusNotAcceptable)
			io.WriteString(w, "incorrect request method expected POST")
			return
		}

		id, _ := uuid.NewRandom()

		state.Write(id, 1)

		w.Header().Add("Location", fmt.Sprintf("/session/%s", id))
		w.WriteHeader(http.StatusSeeOther)
	})

	r.Get("/session/{id}", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			w.WriteHeader(http.StatusNotAcceptable)
			io.WriteString(w, "incorrect request method expected GET")
			return
		}

		id, err := uuid.Parse(chi.URLParam(r, "id"))
		if err != nil {
			http.Error(w, "Not found", 404)
			return
		}

		_, exists := state.Read(id)
		if !exists {
			http.Error(w, "Not found", 404)
			return
		}

		err = t["session"].Execute(w, SessionPage{SessionId: id})
		if err != nil {
			log.Fatal(err)
		}
	})

	r.Mount("/ws/{id}", newSession(&state))

	log.Fatal(http.ListenAndServe(":3000", r))
}

type SessionPage struct {
	SessionId uuid.UUID
}

type T struct {
	Msg  string
	User string
}

func (s *AppState) Read(key uuid.UUID) (int, bool) {
	s.Lock()
	v, e := s.Sessions[key]
	s.Unlock()

	return v, e
}

func (s *AppState) Write(key uuid.UUID, value int) {
	s.Lock()
	s.Sessions[key] = value
	s.Unlock()
}
