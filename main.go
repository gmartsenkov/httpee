package main

import (
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"time"

	"github.com/alecthomas/kong"
	"github.com/samber/lo"
	"strings"
)

type Cmd struct {
	Verbose     bool `help:"Dispay all information about the request and response"`
	ShowHeaders bool `help:"Display response headers"`
}

var CLI struct {
}

var Httpee = []kong.Option{
	kong.Name("HTTPEE"),
	kong.Description("Easy HTTP-ee client"),
	kong.UsageOnError(),
	kong.ConfigureHelp(kong.HelpOptions{
		Compact: true,
	})}

func main() {
	file, err := os.Open("httpee.toml")
	if err != nil {
		fmt.Println("Config file `httpee.toml` not found")
		os.Exit(1)
	}

	var cfg Config
	err = cfg.parse(file)

	if err != nil {
		fmt.Println("Failed to parse httpee.toml", err)
		os.Exit(1)
	}

	for _, filePath := range cfg.AdditionalConfigs {
		file, err := os.Open(filePath)
		if err != nil {
			continue
		}

		var additionalCfg Config
		err = additionalCfg.parse(file)
		if err != nil {
			fmt.Println("Failed to parse ", filePath, err)
			os.Exit(1)
		}

		cfg.merge(&additionalCfg)
	}

	filePaths := lo.FlatMap(cfg.Dirs, func(dir string, _ int) []string {
		templates, _ := findTemplates(dir)
		if err != nil {
			fmt.Printf("Failed to access %s directory\r", dir)
			os.Exit(1)
		}

		return templates
	})

	templates := make(map[string]Template)
	for _, filePath := range filePaths {
		fileName :=
			strings.TrimSuffix(filePath, filepath.Ext(filePath))

		file, err := os.Open(filePath)
		if err != nil {
			fmt.Printf("Failed to read file %s", filePath)
			os.Exit(1)
		}

		var template Template
		err = template.parse(file, &cfg)
		if err != nil {
			fmt.Println("Failed to parse ", filePath, err)
			os.Exit(1)
		}

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

	ctx := kong.Parse(&CLI, append(Httpee, kongCommands...)...)
	cmd := ctx.Command()

	template, found := templates[cmd]
	if !found {
		fmt.Println(cmd)
	}

	command, found := commands[cmd]
	if !found {
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
