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
	body := runTemplate(template.Request.Body, template)
	req, err := http.NewRequest(
		template.Request.Method,
		runTemplate(template.Request.Url, template),
		bytes.NewReader([]byte(body)),
	)

	if err != nil {
		return nil, err
	}

	for key, value := range template.Request.Headers {
		req.Header.Add(key, runTemplate(value, template))
	}

	return client.Do(req)
}
