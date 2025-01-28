package main

import (
	"log"
	"net/http"
	"io"
)

func h1(w http.ResponseWriter, _ *http.Request) {
    io.WriteString(w, index())
}

func main() {
	http.HandleFunc("/", h1)

	// Start the server on port 8080
	log.Println("Starting server on :8080")
	err := http.ListenAndServe(":8080", nil)
	if err != nil {
		log.Fatal("ListenAndServe: ", err)
	}
}
