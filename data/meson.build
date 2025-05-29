blueprints = custom_target(
  'blueprints',
  input: files(
    'blueprint/directory_browser.blp',
    'blueprint/editor.blp',
    'blueprint/tabview.blp',
    'blueprint/toolbar.blp',
    'blueprint/window.blp',
  ),
  output: '.',
  command: [
    find_program('blueprint-compiler'),
    'batch-compile',
    '@OUTPUT@',
    '@CURRENT_SOURCE_DIR@',
    '@INPUT@',
  ],
)
service_conf = configuration_data()
service_conf.set('bindir', get_option('prefix') / get_option('bindir'))
service_conf.set('app_id', APPLICATION_ID)
configure_file(
  input: 'org.gnome.Example.service.in',
  output: 'org.gnome.Example.service',
  configuration: service_conf,
  install_dir: get_option('datadir') / 'dbus-1' / 'services',
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
