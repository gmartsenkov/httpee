package main

import (
	"github.com/stretchr/testify/assert"
	"testing"
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
