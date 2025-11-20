#! /usr/bin/bash

echo "#set page(height: auto, margin: 0cm)

#let N = $2
#let n = 1
#stack(
  while n <= N {
    image(\"pdfs/$1.pdf\", page: n, width: auto)
    n += 1
  }
)" >typst.typ

typst c typst.typ typst.pdf
pdf2svg typst.pdf svgs/$1.svg
rm typst.pdf typst.typ
