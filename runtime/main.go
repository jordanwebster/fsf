package main

import (
	"log"
	"net/http"
)

func main() {
	// Create a file server handler for the current directory
	fs := http.FileServer(http.Dir("../static"))

	// Handle all requests with the file server
	http.Handle("/", fs)

	// Start the server on port 8080
	log.Println("Starting server on :8080")
	err := http.ListenAndServe(":8080", nil)
	if err != nil {
		log.Fatal("ListenAndServe: ", err)
	}
}
