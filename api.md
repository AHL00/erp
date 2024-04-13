# REST API 

## GET /auth/login
- Description: Login to the system
- Request: 
    - username: (string) username
    - password: (string) password
- Response: 
    - 401: Unauthorized [Invalid username or password]
    - 200: OK
        - Creates cookies

## GET /auth/logout
- Description: Logout from the system
- Request: 
    - None
- Response: 
    - 401: Unauthorized [Not logged in]
    - 200: OK
        - Deletes cookies

## GET /auth/status
- Description: Check login status
- Request: 
    - None [Cookies]
- Response: 
    - 401: Unauthorized [Not logged in or invalid cookies]
    - 200: OK 
        ```json
        {
            "username": [string],
            "permissions": [i32]
        }
        ```

## GET /auth/register
- Description: Register a new user
- TODO