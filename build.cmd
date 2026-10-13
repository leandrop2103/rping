@echo off
setlocal
cd /d "%~dp0"
echo [1/2] Compilando perfil Release otimizado...
cargo build --release
if errorlevel 1 exit /b %errorlevel%

echo [2/2] Copiando binarios para a raiz do projeto...
copy /Y target\release\rping.exe rping.exe >nul
copy /Y target\release\ping.exe ping.exe >nul
echo Sucesso! Binarios rping.exe e ping.exe salvos na raiz do projeto.
