package main

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestTemplateInterpolate(t *testing.T) {
	template := Template{
		Variables: map[string]any{
			"id":    1,
			"token": "secret",
		},
	}

	assert.Equal(t, template.interpolate("Hey {{id}}"), "Hey 1")
	assert.Equal(t, template.interpolate("Hey {{token}}"), "Hey secret")
	assert.Equal(t, template.interpolate("Hey {{missing}}"), "Hey ")
}

func TestNormaliseVariables(t *testing.T) {
	template := Template{
		Variables: map[string]any{
			"id":    1,
			"other": 1.5,
			"bool":  true,
			"token": "secret",
		},
	}

	assert.Equal(t, template.normalisedVariables(), map[string]any{
		"id":    "1",
		"other": "1.5",
		"bool":  "true",
		"token": "secret",
	})
}

func TestTemplateParse(t *testing.T) {
	content := `
        name = "Users"
        description = "Create user endpoint"

        [variables]
        id = 100
        token = "123"
        `

	var template Template
	reader := strings.NewReader(content)
	config := Config{
		Variables: map[string]any{
			"id":    1,
			"other": "test",
		},
	}

	assert.Nil(t, template.parse(reader, &config))
	assert.Equal(t, template, Template{
		Name:        "Users",
		Description: "Create user endpoint",
		Variables: map[string]any{
			"id":    int64(100),
			"token": "123",
			"other": "test",
		},
	})
}
