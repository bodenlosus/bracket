
blueprints = custom_target('blueprints',
  input: files(
    'blueprint/directory_browser.blp',
    'blueprint/editor.blp',
    'blueprint/tabview.blp',
    'blueprint/toolbar.blp',
    'blueprint/window.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

gnome.compile_resources(
  PROJECT_RDNN_NAME,
  PROJECT_RDNN_NAME + '.gresource.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  install_dir: PKGDATA_DIR,
  install: true,
  dependencies: blueprints,
)
