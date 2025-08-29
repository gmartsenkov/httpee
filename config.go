package main

import (
	"io"

	"github.com/pelletier/go-toml/v2"
	"github.com/samber/lo"
)

type Config struct {
	Dirs              []string
	Variables         map[string]any
	AdditionalConfigs []string `toml:"additional-configs"`
}

func (conf *Config) merge(other *Config) {
	conf.Variables = lo.Assign(conf.Variables, other.Variables)
	conf.Dirs = lo.Union(conf.Dirs, other.Dirs)
}

func (conf *Config) parse(reader io.Reader) error {
	content, err := io.ReadAll(reader)
	if err != nil {
		return err
	}

	err = toml.Unmarshal(content, conf)
	if err != nil {
		return err
	}

	return nil
}
