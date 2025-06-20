package main

import (
	"log"
	"net/http"
	"io"
	"strings"
)

// HTMLBuilder provides a simple API for building HTML strings
type HTMLBuilder struct {
	elements []string // Stack to track open elements
	buffer   strings.Builder
}

// NewHTMLBuilder creates a new HTMLBuilder instance
func NewHTMLBuilder() *HTMLBuilder {
	return &HTMLBuilder{
		elements: make([]string, 0),
	}
}

// beginElement starts a new HTML element
func (h *HTMLBuilder) beginElement(name string) {
	h.buffer.WriteString("<")
	h.buffer.WriteString(name)
	h.buffer.WriteString(">")
	h.elements = append(h.elements, name)
}

// endElement closes the most recently opened element
func (h *HTMLBuilder) endElement() {
	if len(h.elements) == 0 {
		return // No elements to close
	}

	// Pop the last element from the stack
	lastIndex := len(h.elements) - 1
	elementName := h.elements[lastIndex]
	h.elements = h.elements[:lastIndex]

	// Write the closing tag
	h.buffer.WriteString("</")
	h.buffer.WriteString(elementName)
	h.buffer.WriteString(">")
}

// addString adds text content to the current element
func (h *HTMLBuilder) addString(text string) {
	h.buffer.WriteString(text)
}

// build returns the final HTML string
func (h *HTMLBuilder) build() string {
	return h.buffer.String()
}

// Reset clears the builder for reuse
func (h *HTMLBuilder) Reset() {
	h.elements = h.elements[:0]
	h.buffer.Reset()
}

// TODO: Delete once we have tree pruning. This is to avoid unused variables
// in Go.
func (h *HTMLBuilder) addAttribute(name string, value interface{}) {
}

func h1(w http.ResponseWriter, _ *http.Request) {
    io.WriteString(w, Index())
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
