@echo off
echo Удаление службы MonitorSystemOPs...

:: Проверка прав администратора
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: Требуются права администратора для удаления службы
    pause
    exit /b 1
)

MonitorSystemOPs.exe stop
timeout /t 3 /nobreak >nul
MonitorSystemOPs.exe uninstall

echo Служба успешно удалена
pause