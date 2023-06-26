package main

import (
	"context"
	"log"
	"net/http"
	"sync"
	"time"

	"golang.org/x/time/rate"

	"nhooyr.io/websocket"
)

type Session struct {
	// subscriberMessageBuffer controls the max number
	// of messages that can be queued for a subscriber
	// before it is kicked.
	//
	// Defaults to 16.
	subscriberMessageBuffer int

	// publishLimiter controls the rate limit applied to the publish endpoint.
	//
	// Defaults to one publish every 100ms with a burst of 8.
	publishLimiter *rate.Limiter

	// logf controls where logs are sent.
	// Defaults to log.Printf.
	logf func(f string, v ...interface{})

	// serveMux routes the various endpoints to the appropriate handler.
	serveMux http.ServeMux

	subscribersMu sync.Mutex
	subscribers   map[*subscriber]struct{}
	state         *AppState
}

// newSession constructs a Session with the defaults.
func newSession(state *AppState) *Session {
	session := &Session{
		state:                   state,
		subscriberMessageBuffer: 16,
		logf:                    log.Printf,
		subscribers:             make(map[*subscriber]struct{}),
		publishLimiter:          rate.NewLimiter(rate.Every(time.Millisecond*100), 8),
	}

	session.serveMux.HandleFunc("/", session.subscribeHandler)

	return session
}

// subscriber represents a subscriber.
// Messages are sent on the msgs channel and if the client
// cannot keep up with the messages, closeSlow is called.
type subscriber struct {
	msgs      chan []byte
	closeSlow func()
}

func (session *Session) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	session.serveMux.ServeHTTP(w, r)
}

// subscribeHandler accepts the WebSocket connection and then subscribes
// it to all future messages.
func (session *Session) subscribeHandler(w http.ResponseWriter, r *http.Request) {
	rl := rate.NewLimiter(rate.Every(time.Millisecond*100), 10)
	c, err := websocket.Accept(w, r, nil)
	if err != nil {
		session.logf("%v", err)
		return
	}
	defer c.Close(websocket.StatusInternalError, "")

	go session.subscribe(r.Context(), c)

	for {
		rl.Wait(r.Context())

		_, b, _ := c.Read(r.Context())

		if len(b) > 0 {
			session.publish(b)
		}
	}
}

func (session *Session) subscribe(ctx context.Context, c *websocket.Conn) error {
	s := &subscriber{
		msgs: make(chan []byte, session.subscriberMessageBuffer),
		closeSlow: func() {
			c.Close(websocket.StatusPolicyViolation, "connection too slow to keep up with messages")
		},
	}
	session.addSubscriber(s)
	defer session.deleteSubscriber(s)

	for {
		select {
		case msg := <-s.msgs:
			err := writeTimeout(ctx, time.Second*5, c, msg)
			if err != nil {
				return err
			}
		case <-ctx.Done():
			return ctx.Err()
		}
	}
}

// publish publishes the msg to all subscribers.
// It never blocks and so messages to slow subscribers
// are dropped.
func (session *Session) publish(msg []byte) {
	session.subscribersMu.Lock()
	defer session.subscribersMu.Unlock()

	session.publishLimiter.Wait(context.Background())

	for s := range session.subscribers {
		select {
		case s.msgs <- msg:
		default:
			go s.closeSlow()
		}
	}
}

// addSubscriber registers a subscriber.
func (session *Session) addSubscriber(sub *subscriber) {
	session.subscribersMu.Lock()
	session.subscribers[sub] = struct{}{}
	session.subscribersMu.Unlock()
}

// deleteSubscriber deletes the given subscriber.
func (session *Session) deleteSubscriber(sub *subscriber) {
	session.subscribersMu.Lock()
	delete(session.subscribers, sub)
	session.subscribersMu.Unlock()
}

func writeTimeout(ctx context.Context, timeout time.Duration, c *websocket.Conn, msg []byte) error {
	ctx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	return c.Write(ctx, websocket.MessageText, msg)
}
