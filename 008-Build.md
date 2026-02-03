Okay I want this to be easily buildable
So we need something like `build.sh`
And eventually make a Dockerfile

What are requirements?
npm
Cargo
vite
At the very least

Okay so
we need to check for cargo
if no cargo, install cargo (automatically?)

```sh
#!/bin/bash


install_npm() {
    echo "installing the thingy :3 (node)"
    # this assumes debian based system
    # FIX LATER!!
    curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
    sudo apt-get install -y nodejs
}

install_rust() {
    echo "installing the thingy :3 (cargo)"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env  # changing source makes cargo immediately available
}

install_typescript() {
    echo "Installing the thingy :3 (typescript)"
    npm install -g typescript
}


# Check for required dependencies
echo "Checking dependencies"

# check cargo
if ! command -v cargo &> /dev/null
then
    echo "installing rust now :3"
    install_rust
fi

# check npm
if ! command -v npm &> /dev/null
then
    echo "installing node :D"
    install_npm
fi

# check typescript
if ! command -v tsc &> /dev/null
then
    echo "typescript installing"
    install_typescript
fi

# Install Node.js dependencies
echo "installing node dependencies"
npm install --prefix ./frontend/vite-dev

# Build the engine and API
echo "building engine"
cd engine
cargo build --release
cd ..

echo "building api"
cd api
cargo build --release
cd ..

# Build the front-end
echo "building front-end"
cd frontend/vite-dev
npm run build
cd ../..

echo "Build completed successfully!"
```
