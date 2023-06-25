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
)

type AppState struct {
	sync.Mutex
	Sessions map[string]int
}

type templates map[string]*template.Template

func main() {
	t := templates{
		"index":   template.Must(template.ParseFiles("templates/layout.html", "templates/index.html")),
		"session": template.Must(template.ParseFiles("templates/layout.html", "templates/session.html")),
	}

	state := AppState{
		Sessions: map[string]int{},
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

		id := "test"

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

		id := chi.URLParam(r, "id")
		_, exists := state.Read(id)
		if !exists {
			http.Error(w, "Not found", 404)
			return
		}

		err := t["session"].Execute(w, nil)
		if err != nil {
			log.Fatal(err)
		}
	})

	log.Fatal(http.ListenAndServe(":3000", r))
}

func (s *AppState) Read(key string) (int, bool) {
	s.Lock()
	v, e := s.Sessions[key]
	s.Unlock()

	return v, e
}

func (s *AppState) Write(key string, value int) {
	s.Lock()
	s.Sessions[key] = value
	s.Unlock()
}
