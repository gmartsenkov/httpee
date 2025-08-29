package main

import "github.com/samber/lo"

type Config struct {
	Dirs              []string
	Variables         map[string]any
	AdditionalConfigs []string `toml:"additional-configs"`
}

func (conf *Config) merge(other *Config) {
	conf.Variables = lo.Assign(conf.Variables, other.Variables)
	conf.Dirs = lo.Union(conf.Dirs, other.Dirs)
}
