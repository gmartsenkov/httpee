package main

import (
	"bytes"
	"fmt"
	"net/http"
	"sort"

	"github.com/alecthomas/chroma/v2/quick"
	"github.com/samber/lo"
)

func sortedHeader(headers http.Header) []string {
	keys := lo.Keys(headers)
	sort.Strings(keys)

	result := make([]string, 0, len(keys))

	for _, key := range keys {
		value := headers.Get(key)
		str := fmt.Sprintf("%s: \"%s\"", key, value)
		result = append(result, str)
	}

	return result
}

func highlightBody(contentType string, body []byte) string {
	var buf bytes.Buffer
	var lexer string
	switch contentType {
	case "text/html":
		lexer = "html"
	case "application/json":
		lexer = "json"
	default:
		return string(body)
	}
	_ = quick.Highlight(&buf, string(body), lexer, "terminal256", "tokyonight-night")

	return buf.String()
}
