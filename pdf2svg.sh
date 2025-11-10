#! /usr/bin/bash

echo "#set page(height: auto)

#let N = $2
#let n = 1
#stack(
  while n <= N {
    image(\"pdfs/$1.pdf\", page: n, width: auto)
    n += 1
  }
)" > typst.typ
# use $1 instead of pdfs/$1, even though pdfs will always be in pdfs/, to allow shell autocompletion for paths

typst c typst.typ typst.pdf
pdf2svg typst.pdf svgs/$1.svg
rm typst.pdf typst.typ
