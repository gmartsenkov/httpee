package main

import (
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"

	"strings"

	"github.com/alecthomas/kong"
	"github.com/pelletier/go-toml/v2"
)

type Config struct {
	Dirs      []string
	Variables map[string]string
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
	Variables   map[string]string
	Request     Request
}

type Cmd struct {
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
		file, err := os.ReadFile(filePath)
		if err != nil {
			fmt.Printf("Failed to read file %s", filePath)
			os.Exit(1)
		}

		var template Template
		err = toml.Unmarshal([]byte(file), &template)
		if err != nil {
			fmt.Printf("Failed to parse %s", file)
			os.Exit(1)
		}

		templates[filePath] = template
	}

	// fmt.Printf("%+v\n", templates)
	var commands []kong.Option

	for filePath, template := range templates {
		var cmd Cmd

		one := kong.DynamicCommand(filePath, template.Description, template.Name, &cmd)
		commands = append(commands, one)
	}

	options := []kong.Option{
		kong.Name("HTTPEE"),
		kong.Description("Easy HTTP-ee client"),
		kong.UsageOnError(),
		kong.ConfigureHelp(kong.HelpOptions{
			Compact: true,
		})}

	ctx := kong.Parse(&CLI, append(options, commands...)...)
	cmd := ctx.Command()

	template, found := templates[cmd]
	if found != true {
		fmt.Println(cmd)
	}

	resp, err := make_request(template)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	fmt.Println(resp)
	body, _ := io.ReadAll(resp.Body)
	if err == nil {
		fmt.Println(string(body))
	}
}

func make_request(template Template) (*http.Response, error) {
	client := http.DefaultClient
	req, err := http.NewRequest(template.Request.Method, template.Request.Url, nil)
	if err != nil {
		return nil, err
	}

	for key, value := range template.Request.Headers {
		req.Header.Add(key, value)
	}

	return client.Do(req)
}
