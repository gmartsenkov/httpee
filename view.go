package main

import (
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/list"
	"github.com/charmbracelet/bubbles/stopwatch"
	"github.com/charmbracelet/bubbles/viewport"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/samber/lo"
)

var (
	docStyle      = lipgloss.NewStyle().Margin(1, 2)
	selectedStyle = lipgloss.NewStyle().Padding(0, 3)
	subtleStyle   = lipgloss.NewStyle().Foreground(lipgloss.Color("#7A7573")).Italic(true)
)

var keys = keyMap{
	Enter: key.NewBinding(
		key.WithKeys("submit", "enter"),
		key.WithHelp("enter", "submit"),
	),
	Quit: key.NewBinding(
		key.WithKeys("q", "esc", "ctrl+c"),
		key.WithHelp("q", "quit"),
	),
}

var keysWithResponse = keyMap{
	Enter: key.NewBinding(
		key.WithKeys("re-submit", "enter"),
		key.WithHelp("enter", "re-submit"),
	),
	Quit: key.NewBinding(
		key.WithKeys("q", "esc", "ctrl+c"),
		key.WithHelp("q", "quit"),
	),
	Up: key.NewBinding(
		key.WithKeys("up", "k"),
		key.WithHelp("↑/k", "move up"),
	),
	Down: key.NewBinding(
		key.WithKeys("down", "j"),
		key.WithHelp("↓/j", "move down"),
	),
}

type keyMap struct {
	Enter key.Binding
	Quit  key.Binding
	Up    key.Binding
	Down  key.Binding
}

func (k keyMap) ShortHelp() []key.Binding {
	return []key.Binding{k.Enter, k.Up, k.Down, k.Quit}
}

func (k keyMap) FullHelp() [][]key.Binding {
	return [][]key.Binding{
		{k.Enter, k.Quit},
	}
}

type item struct {
	template *Template
}

func (i item) Title() string {
	if len(i.template.Description) > 0 {
		return i.template.Name + " " + subtleStyle.Render("("+i.template.Description+")")
	}

	return i.template.Name

}
func (i item) Description() string {
	return subtleStyle.Render("POST /users/create/{id}")
}
func (i item) FilterValue() string { return i.template.Name }

type responseMessage struct {
	response *http.Response
	body     []byte
}

type model struct {
	help             help.Model
	list             list.Model
	width            int
	stopwatch        stopwatch.Model
	selected         *item
	response         *responseMessage
	responseViewPort viewport.Model
}

func (m model) Init() tea.Cmd {
	return nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	if m.selected != nil {
		return selectedUpdate(m, msg)
	}

	return listUpdate(m, msg)
}

func selectedUpdate(m model, msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		keypress := msg.String()
		if m.selected != nil {
			switch keypress {
			case "q", "ctrl+c", "esc":
				m.selected = nil
				m.response = nil

				return m, m.stopwatch.Stop()

			case "enter":
				m.response = nil
				return m, tea.Sequence(
					m.stopwatch.Reset(),
					m.stopwatch.Start(),
					makeRequest(m.selected.template),
					m.stopwatch.Stop(),
				)
			}
		}
	case responseMessage:
		m.response = &msg
		responseBody := highlightBody(
			msg.response.Header.Get("Content-Type"),
			msg.body,
		)
		m.responseViewPort.SetContent(responseBody)

	case tea.WindowSizeMsg:
		h, v := selectedStyle.GetFrameSize()
		m.list.SetSize(msg.Width-h, msg.Height-v)
		m.responseViewPort.Width = msg.Width - h
		m.width = msg.Width - h
	}

	var stopwatchCmd tea.Cmd
	var viewPortCmd tea.Cmd
	m.stopwatch, stopwatchCmd = m.stopwatch.Update(msg)
	m.responseViewPort, viewPortCmd = m.responseViewPort.Update(msg)
	return m, tea.Batch(stopwatchCmd, viewPortCmd)
}

func makeRequest(template *Template) tea.Cmd {
	return func() tea.Msg {
		req, _ := template.newHttpRequest()
		resp, _ := http.DefaultClient.Do(req)
		body, _ := io.ReadAll(resp.Body)

		return responseMessage{response: resp, body: body}
	}
}

func listUpdate(m model, msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		keypress := msg.String()
		switch keypress {
		case "q", "ctrl+c":
			return m, tea.Quit

		case "enter":
			i, ok := m.list.SelectedItem().(item)
			if ok {
				m.selected = &i
			}
		}
	case tea.WindowSizeMsg:
		h, v := docStyle.GetFrameSize()
		m.list.SetSize(msg.Width-h, msg.Height-v)
		m.responseViewPort.Width = msg.Width - h
	}
	var cmd tea.Cmd
	m.list, cmd = m.list.Update(msg)
	return m, cmd
}

func (m model) View() string {
	if m.selected != nil {
		resp := ""
		elapsed := ""
		helpView := m.help.View(keys)
		if m.response != nil {
			helpView = m.help.View(keysWithResponse)
			resp = renderResponse(&m)
		}
		if m.response != nil || m.stopwatch.Running() {
			elapsed = lipgloss.
				NewStyle().
				Padding(1, 0).
				Foreground(m.help.Styles.ShortDesc.GetForeground()).
				Render("Elapsed: " + m.stopwatch.View())
		}

		return selectedStyle.Width(m.width).Render(
			lipgloss.JoinVertical(
				lipgloss.Left,
				renderTemplate(m.selected),
				resp,
				elapsed,
				helpView,
			),
		)
	}

	return docStyle.Render(m.list.View())
}

func renderResponse(m *model) string {
	response := m.response.response

	return lipgloss.JoinVertical(
		lipgloss.Left,
		("Status: " + response.Status),
		"Headers:",
		lipgloss.JoinVertical(lipgloss.Left, sortedHeader(response.Header)...),
		"Body:",
		m.responseViewPort.View(),
	)
}

func renderTemplate(i *item) string {
	st := lipgloss.NewStyle().
		Padding(1)

	req, _ := i.template.newHttpRequest()
	body, _ := io.ReadAll(req.Body)

	reqHeaders := sortedHeader(req.Header)

	method := lipgloss.
		NewStyle().
		Padding(0, 1).
		Bold(true).
		Background(lipgloss.Color("#7D56F4")).
		Render(req.Method)

	url := lipgloss.
		NewStyle().
		Padding(0, 1).
		Italic(true).
		Background(lipgloss.Color("#3816A6")).
		Render(req.URL.String())

	header := lipgloss.JoinHorizontal(
		lipgloss.Left,
		method,
		url,
	)

	bodyStyle := lipgloss.
		NewStyle().
		Foreground(lipgloss.Color("#B2BEB5")).
		Border(lipgloss.NormalBorder(), true, false, false)

	return st.Render(lipgloss.JoinVertical(
		lipgloss.Left,
		header,
		bodyStyle.Render(lipgloss.JoinVertical(lipgloss.Left, reqHeaders...)),
		bodyStyle.Render(strings.TrimSpace(string(body))),
	))
}
func program(templates []Template) *tea.Program {
	items := lo.Map(templates, func(t Template, _ int) list.Item {
		return item{template: &t}
	})
	m := model{
		help:             help.New(),
		responseViewPort: viewport.New(40, 10),
		stopwatch:        stopwatch.NewWithInterval(time.Millisecond),
		list:             list.New(items, list.NewDefaultDelegate(), 0, 0),
	}
	m.list.Title = "Pick a request from the collection"

	return tea.NewProgram(m, tea.WithAltScreen())
}
