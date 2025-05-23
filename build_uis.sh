echo "building blueprints"
blueprint-compiler batch-compile zennote/resources zennote/ui $(find zennote/ui/ -name '*.blp' -printf "%p ")
echo "compiling resources"

glib-compile-resources zennote/resources/ui.gresource.xml --sourcedir="zennote/resources/"
# glib-compile-schemas data
