import subprocess as sp

sp.check_call('mkdir dist', shell=True)
tags = sp.check_output('git tag', shell=True).decode('utf-8').strip().split('\n')
for tag in tags:
    sp.check_call(f'git checkout tags/{tag} && cargo make build-web && cp -r wasm dist/{tag} && git stash', shell=True)
