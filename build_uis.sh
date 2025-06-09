blueprint-compiler batch-compile data/ui data/blueprint $(find data/blueprint/*.blp)
glib-compile-resources data/io.github.bracket.gresource.xml --sourcedir=data
