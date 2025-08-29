package main

import (
	"github.com/stretchr/testify/assert"
	"testing"
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
