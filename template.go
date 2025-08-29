package main

import "fmt"

type Template struct {
	Name        string
	Description string
	Variables   map[string]any
	Request     Request
}

func (t *Template) normalisedVariables() map[string]any {
	result := make(map[string]any, len(t.Variables))
	for k, v := range t.Variables {
		result[k] = fmt.Sprintf("%v", v)
	}

	return result
}
