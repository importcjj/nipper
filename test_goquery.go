package main

import (
	"fmt"
	"strings"

	"github.com/PuerkitoBio/goquery"
)

func main() {
	var html = `<div><h1 class="foo">Hello, <i>world!</i></h1></div>`

	doc, err := goquery.NewDocumentFromReader(strings.NewReader(html))

	fmt.Println(err)

	doc.Find("i").Remove()

	html, _ = doc.Html()
	fmt.Println(html)
}