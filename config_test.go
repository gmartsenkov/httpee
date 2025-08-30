package main

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestMergeConfig(t *testing.T) {
	conf1 := Config{
		Dirs: []string{"accounts", "users"},
		Variables: map[string]any{
			"id":    1,
			"token": "secret",
		},
	}
	conf2 := Config{
		Dirs: []string{"accounts", "subscriptions"},
		Variables: map[string]any{
			"token": "override",
		},
	}

	conf1.merge(&conf2)
	assert.Equal(
		t, conf1, Config{
			Dirs: []string{"accounts", "users", "subscriptions"},
			Variables: map[string]any{
				"id":    1,
				"token": "override",
			},
		})
}

func TestParseConfig(t *testing.T) {
	content := `
        dirs = ["example"]
        additional-configs = ["httpee.local.toml"]

        [variables]
        id = 5
        token = "123"
        `

	var config Config
	reader := strings.NewReader(content)

	assert.Nil(t, config.parse(reader))
	assert.Equal(t, config, Config{
		Dirs:              []string{"example"},
		AdditionalConfigs: []string{"httpee.local.toml"},
		Variables: map[string]any{
			"id":    int64(5),
			"token": "123",
		},
	})
}
