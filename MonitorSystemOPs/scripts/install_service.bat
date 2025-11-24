@echo off
echo Установка службы MonitorSystemOPs...

:: Проверка прав администратора
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo ERROR: Требуются права администратора для установки службы
    pause
    exit /b 1
)

:: Создание директорий
if not exist "config" mkdir config
if not exist "logs" mkdir logs
if not exist "data" mkdir data

:: Генерация конфигурации
MonitorSystemOPs.exe config

:: Установка службы
MonitorSystemOPs.exe install

:: Запуск службы
MonitorSystemOPs.exe start

echo.
echo ========================================
echo Служба успешно установлена и запущена!
echo ========================================
echo.
echo Управление службой:
echo   Остановка: MonitorSystemOPs.exe stop
echo   Запуск:    MonitorSystemOPs.exe start
echo   Перезапуск: MonitorSystemOPs.exe restart
echo   Удаление:   MonitorSystemOPs.exe uninstall
echo.
echo Веб-интерфейс: http://localhost:8080
echo.
pause