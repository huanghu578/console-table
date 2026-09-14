@echo off
title One-Click Sync Folder

echo ========================================
echo Starting folder sync to github...
echo ========================================

:: 1. Check if current directory is a Git repository
if not exist ".git" (
    echo Error: Current directory is not a Git repository!
    echo Please run git init and configure remote repository first.
    pause
    exit /b 1
)

:: 2. Get current branch name
for /f %%i in ('git branch --show-current') do set current_branch=%%i
if "%current_branch%"=="" (
    echo Error: Unable to detect current branch. Are you in a Git repository?
    pause
    exit /b 1
)
echo Current branch: %current_branch%

:: 3. Get current time as commit message
for /f "tokens=1-3 delims=/- " %%a in ('date /t') do (
    set date_str=%%a-%%b-%%c
)
for /f "tokens=1-2 delims=: " %%a in ('time /t') do (
    set time_str=%%a:%%b
)
set commit_msg=Sync on %date_str% %time_str%

:: 4. Add all changes
echo Adding all changes...
git add .
if %errorlevel% neq 0 (
    echo git add failed, please check file permissions or Git status.
    pause
    exit /b %errorlevel%
)

:: 5. Commit to local
echo Committing to local repository...
git commit -m "%commit_msg%"
if %errorlevel% neq 0 (
    if %errorlevel% equ 1 (
        echo No changes to commit, skipping commit step.
    ) else (
        echo git commit failed, please check error messages.
        pause
        exit /b %errorlevel%
    )
)

:: 6. Pull latest from remote (using rebase)
echo Pulling latest from remote...
git pull --rebase origin %current_branch%
if %errorlevel% neq 0 (
    echo ========================================
    echo Conflict detected during pull!
    echo Please resolve conflicts manually, then run git rebase --continue
    echo or use git merge --abort to cancel this operation.
    echo ========================================
    pause
    exit /b %errorlevel%
)

:: 7. Push to remote
echo Pushing to remote...
git push origin %current_branch%
if %errorlevel% neq 0 (
    echo git push failed, please check network or permissions.
    pause
    exit /b %errorlevel%
)

echo ========================================
echo Sync completed!
echo Branch: %current_branch%
echo Commit message: %commit_msg%
echo ========================================
pause