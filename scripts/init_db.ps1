
if ($null -eq (Get-Command "psql.exe" -ErrorAction SilentlyContinue)) 
{ 
    Write-Error "Unable to find psql in your PATH" -ErrorAction Stop
}

if ($null -eq (Get-Command "sqlx.exe" -ErrorAction SilentlyContinue)) 
{ 
    Write-Error "Unable to find sqlx in your PATH" -ErrorAction Stop
}

if ($null -eq (Get-Command "docker.exe" -ErrorAction SilentlyContinue)) 
{ 
    Write-Error "Unable to find docker in your PATH" -ErrorAction Stop
}

$DB_USER = $POSTGRES_USER ?? 'postgres'
$DB_PASSWORD = $POSTGRES_PASSWORD ?? 'my_very_secure_database_password_1'
$DB_NAME = $POSTGRES_DB ?? 'newsletter'
$DB_PORT = $POSTGRES_PORT ?? '5432'

$env:DATABASE_URL = "postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"

if (Get-Variable 'SKIP_DOCKER' -Scope 'Global' -ErrorAction 'Ignore') {
    Write-Host "Skipping Docker launch..."
} else {
    docker run `
        -e POSTGRES_USER=${DB_USER} `
        -e POSTGRES_PASSWORD=${DB_PASSWORD} `
        -e POSTGRES_DB=${DB_NAME} `
        -p "${DB_PORT}":5432 `
        -d postgres `
        postgres -N 1000
}

$env:PGPASSWORD="${DB_PASSWORD}"

do {
    Write-Output "Waiting for postgres database to come online..."
    Start-Sleep -Seconds 1.0
} until (psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q')


cargo sqlx database create
cargo sqlx migrate run
