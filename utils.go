package main

import (
	"fmt"
	"net/http"
	"sort"

	"github.com/samber/lo"
)

func sortedHeader(headers http.Header) []string {
	keys := lo.Keys(headers)
	sort.Strings(keys)

	result := make([]string, 0, len(keys))

	for _, key := range keys {
		value := headers.Get(key)
		str := fmt.Sprintf("\"%s\": \"%s\"", key, value)
		result = append(result, str)
	}

	return result
}
