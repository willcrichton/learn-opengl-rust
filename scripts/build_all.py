import subprocess as sp

sp.check_call('mkdir dist && cp -r assets dist && rm -rf dist/assets/shaders', shell=True)

tags = sp.check_output('git tag', shell=True).decode('utf-8').strip().split('\n')
for tag in tags:
    sp.check_call(f''' git checkout tags/{tag} && \
    cargo make build-web --profile release && \
    rm -f wasm/assets && \
    cp -r wasm dist/{tag} && \
    mkdir dist/{tag}/assets && \
    cd dist/{tag}/assets && \
    ln -s ../../assets/* . && \
    (cp -r assets/shaders dist/{tag}/assets ||: ) && \
    git stash''', 
      shell=True)
