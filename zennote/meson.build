blueprint_compiler = find_program('blueprint-compiler', required: true)

blueprint_sources = files('resources/window.blp')

blp_generator = custom_target(
  'blueprint-files',
  input: blueprint_sources,
  command: [blueprint_compiler, 'compile', '@INPUT@', '--output', '@OUTPUT@'],
  output: '@BASENAME@.ui',
)

generated_ui_files = blp_generator

gresource = gnome.compile_resources(
  'bracket',
  'resources/resources.gresource.xml',
  dependencies: [generated_ui_files],
)
