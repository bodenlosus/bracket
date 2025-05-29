project(
  'zennote',
  version: '0.1.0',
  meson_version: '>= 1.0.0',
  default_options: [
    'warning_level=2',
    'werror=false',
  ],
)
gnome = import('gnome')
python = import('python')

py_installation = python.find_installation('python3')

# Make sure Python is installed and found
if not py_installation.found()
  error('No valid python3 binary found')
endif

if not py_installation.language_version().version_compare('>= 3.9')
  error('Python 3.9 or newer is required.')
endif

PROJECT_RDNN_NAME = 'io.github.johannes.zennote'
APPLICATION_ID = PROJECT_RDNN_NAME
PKGDATA_DIR = join_paths(get_option('prefix'), get_option('datadir'), APPLICATION_ID)
PKGLIB_DIR = join_paths(get_option('prefix'), get_option('libdir'), APPLICATION_ID)
PYTHON_DIR = py_installation.get_path('purelib')

glib_dep = dependency('glib-2.0', version: '>= 2.67.1')
dependency('gobject-introspection-1.0', version: '>= 1.35.0')
dependency('gtk4', version: '>= 4.16.0')
dependency('libadwaita-1', version: '>= 1.7', method: 'pkg-config')
dependency('pango', version: '>= 1.44.0')
dependency('pygobject-3.0', version: '>= 3.50.0')
dependency('py3cairo', version: '>= 1.14.0')

subdir('data')

moduledir = PKGDATA_DIR / 'zennote'
install_subdir('zennote', install_dir: moduledir)

run_command(
  py_installation,
  '-m', 'pip',
  'install',
  '--prefix', get_option('prefix'),
  'wheels/highlighter-0.1.0-cp312-cp312-linux_x86_64.whl',
  check: true,
)

# Install the executable file
bin_config = configuration_data()
bin_config.set('application_id', APPLICATION_ID)
bin_config.set('rdnn_name', PROJECT_RDNN_NAME)
bin_config.set('pkgdatadir', PKGDATA_DIR)
bin_config.set('python', py_installation.full_path())
# bin_config.set('localedir', join_paths(get_option('prefix'), get_option('datadir'), 'locale'))
# bin_config.set('pythondir', PYTHON_DIR)
# bin_config.set('schemasdir', PKGDATA_DIR)
# bin_config.set('version', meson.project_version() + VCS_TAG)

configure_file(input: 'manifest.in', output: PROJECT_RDNN_NAME + '.json')

configure_file(
  input: 'zennote.py.in',
  output: 'znte',
  configuration: bin_config,
  install_dir: get_option('bindir'),
)

gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  # update_desktop_database: true
)
