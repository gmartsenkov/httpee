package main

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/samber/lo"
	"strings"
)

type Cmd struct {
	Verbose     bool `help:"Dispay all information about the request and response"`
	ShowHeaders bool `help:"Display response headers"`
}

var CLI struct {
}

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

	lo.Values(templates)
	p := program(lo.Values(templates))
	if _, err := p.Run(); err != nil {
		fmt.Println("Error running program:", err)
		os.Exit(1)
	}
}
