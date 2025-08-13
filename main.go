package main

import (
	"bytes"
	"fmt"
	"io"
	"maps"
	"net/http"
	"os"
	"path/filepath"
	"time"

	"strings"

	"github.com/alecthomas/kong"
	"github.com/pelletier/go-toml/v2"
	"github.com/valyala/fasttemplate"
)

type Config struct {
	Dirs      []string
	Variables map[string]any
}

type Request struct {
	Url     string
	Method  string
	Body    string
	Headers map[string]string
}

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

type Cmd struct {
	Verbose     bool `help:"Dispay all information about the request and response"`
	ShowHeaders bool `help:"Display response headers"`
}

var CLI struct {
}

func main() {
	file, err := os.ReadFile("httpee.toml")
	if err != nil {
		fmt.Println("Config file `httpee.toml` not found")
		os.Exit(1)
	}

	var cfg Config
	err = toml.Unmarshal(file, &cfg)
	if err != nil {
		fmt.Println("Failed to parse httpee.toml")
		os.Exit(1)
	}

	filePaths := []string{}

	for _, dir := range cfg.Dirs {
		dir_entries, err := os.ReadDir(dir)

		if err != nil {
			fmt.Printf("Failed to access %s directory\r", dir)
			os.Exit(1)
		}

		for _, entry := range dir_entries {
			if strings.HasSuffix(entry.Name(), ".toml") != true {
				continue
			}
			filepath := filepath.Join(dir, entry.Name())
			filePaths = append(filePaths, filepath)
		}
	}

	templates := make(map[string]Template)
	for _, filePath := range filePaths {
		fileName :=
			strings.TrimSuffix(filePath, filepath.Ext(filePath))

		file, err := os.ReadFile(filePath)
		if err != nil {
			fmt.Printf("Failed to read file %s", filePath)
			os.Exit(1)
		}

		var template Template
		err = toml.Unmarshal([]byte(file), &template)
		if err != nil {
			fmt.Printf("Failed to parse '%s' \n", filePath)
			fmt.Println(err)
			os.Exit(1)
		}

		template.Variables = mergeMap(template.Variables, cfg.Variables)
		templates[fileName] = template
	}

	var kongCommands []kong.Option
	commands := make(map[string]*Cmd)

	for fileName, template := range templates {
		var cmd Cmd

		one := kong.DynamicCommand(fileName, template.Description, template.Name, &cmd)
		kongCommands = append(kongCommands, one)
		commands[fileName] = &cmd
	}

	options := []kong.Option{
		kong.Name("HTTPEE"),
		kong.Description("Easy HTTP-ee client"),
		kong.UsageOnError(),
		kong.ConfigureHelp(kong.HelpOptions{
			Compact: true,
		})}

	ctx := kong.Parse(&CLI, append(options, kongCommands...)...)
	cmd := ctx.Command()

	template, found := templates[cmd]
	if found != true {
		fmt.Println(cmd)
	}

	command, found := commands[cmd]
	if found != true {
		fmt.Println(cmd)
		os.Exit(1)
	}

	startTime := time.Now()
	resp, err := make_request(&template)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	responseTime := time.Since(startTime).Milliseconds()
	log_response(resp, command)
	fmt.Printf("Response time: %d ms\n", responseTime)
}

func log_response(resp *http.Response, cmd *Cmd) {
	fmt.Println(resp.Request.Method, resp.Request.URL)
	fmt.Println("Status:", resp.Status)

	if cmd.ShowHeaders {
		for k, v := range resp.Header {
			value := strings.Join(v[:], ",")
			fmt.Printf("%s: %s\n", k, value)
		}
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return
	}

	if len(body) > 0 {
		fmt.Println("Body:")
		fmt.Println(string(body))
	}
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

func runTemplate(temp string, template *Template) string {
	t := fasttemplate.New(temp, "{{", "}}")
	return t.ExecuteString(template.normalisedVariables())
}

func mergeMap(map1 map[string]any, map2 map[string]any) map[string]any {
	result := make(map[string]any)
	maps.Copy(result, map2)
	maps.Copy(result, map1)
	return result
}
