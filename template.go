package main

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"

	"github.com/pelletier/go-toml/v2"
	"github.com/samber/lo"
	"github.com/valyala/fasttemplate"
)

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

func (t *Template) interpolate(str string) string {
	engine := fasttemplate.New(str, "{{", "}}")
	return engine.ExecuteString(t.normalisedVariables())
}

func (t *Template) parse(reader io.Reader, conf *Config) error {
	content, err := io.ReadAll(reader)
	if err != nil {
		return err
	}

	err = toml.Unmarshal(content, t)
	if err != nil {
		return err
	}

	t.Variables = lo.Assign(t.Variables, conf.Variables)

	return nil
}
func findTemplates(dir string) ([]string, error) {
	dir_entries, err := os.ReadDir(dir)

	if err != nil {
		return []string{}, nil
	}

	filePaths := []string{}
	for _, entry := range dir_entries {
		if !strings.HasSuffix(entry.Name(), ".toml") {
			continue
		}
		filepath := filepath.Join(dir, entry.Name())
		filePaths = append(filePaths, filepath)
	}

	return filePaths, nil
}
