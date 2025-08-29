package main

import (
	"bytes"
	"net/http"
)

type Request struct {
	Url     string
	Method  string
	Body    string
	Headers map[string]string
}

func make_request(template *Template) (*http.Response, error) {
	client := http.DefaultClient
	body := template.interpolate(template.Request.Body)
	req, err := http.NewRequest(
		template.Request.Method,
		template.interpolate(template.Request.Url),
		bytes.NewReader([]byte(body)),
	)

	if err != nil {
		return nil, err
	}

	for key, value := range template.Request.Headers {
		req.Header.Add(key, template.interpolate(value))
	}

	return client.Do(req)
}
