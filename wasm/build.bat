wasm-pack build
xcopy pkg www\pkg\ /E /Y
DEL www\pkg\.gitignore
cd www
CMD /C npm install
cd ..
CMD /C npm run start --prefix www
