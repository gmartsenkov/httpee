import gleam/io
import gleam/list
import gleam/string
import gleam_community/ansi
import httpee/template.{type Template}

pub fn render(templates: List(Template)) {
  list.each(templates, fn(template) {
    let row = [
      "* ",
      "[",
      template.request_details.method,
      "]",
      " ",
      template.name,
      " ",
      ansi.grey("(" <> template.file_name <> ")"),
    ]

    row
    |> string.concat
    |> io.println()
  })
}
