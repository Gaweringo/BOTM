$taskPath = "\Gaweringo\"
$name = 'BOTM'
$exe = 'botm.exe'
$params = '--no-gui --last-month'
$location = "P:\Martin\Programming\Rust\bangers-of-the-month\target\debug"
# $location = $args[0]
# param ($location)

"lol" 

$action = New-ScheduledTaskAction -Execute "$location\$exe" -Argument "$params" -WorkingDirectory $location
$settings = New-ScheduledTaskSettingsSet -RunOnlyIfNetworkAvailable -StartWhenAvailable

# Use schtasks to create the task, as monthy is not availible in New-ScheduledTaskTrigger
schtasks.exe /Create /SC MONTHLY /TN $taskPath$name /ST 00:01 /TR "$location\$exe --no-gui --last-month" /F 

# Update the created task
Set-ScheduledTask -TaskName $name -TaskPath $taskPath -Action $action -Settings $settings | Out-Null