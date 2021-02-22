import subprocess as sp
from jinja2 import Template

sp.check_call('mkdir dist && cp -r assets dist && rm -rf dist/assets/shaders', shell=True)

tags = sp.check_output('git tag', shell=True).decode('utf-8').strip().split('\n')

template = Template(open('scripts/gh-pages.html').read())
output = template.render(tags=[
  {
    'key': tag[3:],
    'name': tag,
    'orig': tag
  }
  for tag in tags
])
with open('dist/index.html', 'w') as f:
  f.write(output)


for tag in tags:
    sp.check_call(f''' git checkout tags/{tag} && \
    cargo make build-web --profile release && \
    rm -f wasm/assets && \
    cp -r wasm dist/{tag} && \
    mkdir dist/{tag}/assets && \
    cd dist/{tag}/assets && \
    ln -s ../../assets/* . && \
    cd - && \
    (cp -r assets/shaders dist/{tag}/assets ||: ) && \
    git stash''', 
      shell=True)
