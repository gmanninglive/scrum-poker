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
	Sessions map[uuid.UUID]Session
}

type Member struct {
	id   int
	name string
}

type Session struct {
	host Member
}

type templates map[string]*template.Template

type NotFoundCtx struct {
	Status  int
	Title   string
	Message string
}

func main() {
	t := templates{
		"index":   template.Must(template.ParseFiles("templates/layout.html", "templates/index.html")),
		"session": template.Must(template.ParseFiles("templates/layout.html", "templates/session.html")),
		"error":   template.Must(template.ParseFiles("templates/layout.html", "templates/error.html")),
	}

	state := AppState{
		Sessions: map[uuid.UUID]Session{},
	}

	router := chi.NewRouter()
	router.Use(middleware.Logger)
	router.Use(middleware.Recoverer)

	router.Get("/", func(w http.ResponseWriter, r *http.Request) {
		err := t["index"].Execute(w, nil)
		if err != nil {
			log.Print(err.Error())
			http.Error(w, "internal server error", 500)
		}
	})

	router.Post("/session/create", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			w.WriteHeader(http.StatusNotAcceptable)
			io.WriteString(w, "incorrect request method expected POST")
			return
		}

		id, _ := uuid.NewRandom()

		r.ParseForm()
		state.Write(id, Session{host: Member{id: 1, name: r.Form.Get("name")}})

		w.Header().Add("Location", fmt.Sprintf("/session/%s", id))
		w.WriteHeader(http.StatusSeeOther)
	})

	router.Get("/session/{id}", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			w.WriteHeader(http.StatusNotAcceptable)
			io.WriteString(w, "incorrect request method expected GET")
			return
		}

		id, err := uuid.Parse(chi.URLParam(r, "id"))
		if err != nil {
			handleNotFound(w, r)
			return
		}

		_, exists := state.Read(id)
		if !exists {
			handleNotFound(w, r)
			return
		}

		err = t["session"].Execute(w, SessionPageCtx{SessionId: id})
		if err != nil {
			log.Fatal(err)
		}
	})

	router.Mount("/ws/{id}", newSession(&state))

	router.NotFound(handleNotFound)

	router.Get("/404", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(404)
		err := t["error"].Execute(w, NotFoundCtx{Status: 404, Title: "Page not found", Message: "Sorry, we couldn't find the page you're looking for."})
		if err != nil {
			log.Print(err.Error())
			http.Error(w, "internal server error", 500)
		}
	})

	router.Get("/500", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(500)
		err := t["error"].Execute(w, NotFoundCtx{Status: 500, Title: "Internal server error", Message: "Sorry, an error occured on the server."})
		if err != nil {
			log.Print(err.Error())
			http.Error(w, "internal server error", 500)
		}
	})

	log.Fatal(http.ListenAndServe(":3000", router))
}

func handleNotFound(w http.ResponseWriter, r *http.Request) {
	http.Redirect(w, r, "/404", http.StatusSeeOther)
}

type SessionPageCtx struct {
	SessionId uuid.UUID
}

func (s *AppState) Read(key uuid.UUID) (Session, bool) {
	s.Lock()
	v, e := s.Sessions[key]
	s.Unlock()

	return v, e
}

func (s *AppState) Write(key uuid.UUID, value Session) {
	s.Lock()
	s.Sessions[key] = value
	s.Unlock()
}
