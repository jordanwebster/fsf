package main

import (
	"log"
	"net/http"
	"strings"
	"html/template"
)

const htmlTemplate = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{.Title}}</title>
    {{if .InitialData}}
    <script>
        window.__INITIAL_DATA__ = {{.InitialData}};
    </script>
    {{end}}
</head>
<body>
    <div id="root">{{.ServerRenderedContent}}</div>
    <!-- Single bundle with everything included -->
    <script type="module" src="/static/bundle.js"></script>
</body>
</html>`

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

func h1(w http.ResponseWriter, r *http.Request) {
    tmpl := template.Must(template.New("page").Parse(htmlTemplate))
//     io.WriteString(w, Index())
    serverContent := renderCurrentRoute(r.URL.Path)

    // Optional: Add any initial data
    initialData := getInitialDataForRoute(r.URL.Path)

    data := struct {
        Title                 string
        ServerRenderedContent template.HTML
        InitialData          template.JS
    }{
        Title:                 "FSF App",
        ServerRenderedContent: template.HTML(serverContent),
        InitialData:          template.JS(initialData),
    }

    w.Header().Set("Content-Type", "text/html")
    tmpl.Execute(w, data)
}

func renderCurrentRoute(path string) string {
	switch path {
	case "/":
        return Index();
	default:
		return `<div>404 - Page not found</div>`
	}
}

// Optional: Get initial data for hydration
func getInitialDataForRoute(path string) string {
	// TODO: Return any initial data as JSON string
	return `{"timestamp": "` + "2025-06-11T12:00:00Z" + `"}`
}

func main() {
    http.Handle("/static/", http.StripPrefix("/static/", http.FileServer(http.Dir("./javascript/dist/"))))
	http.HandleFunc("/", h1)

	// Start the server on port 8080
	log.Println("Starting server on :8080")
	err := http.ListenAndServe(":8080", nil)
	if err != nil {
		log.Fatal("ListenAndServe: ", err)
	}
}
