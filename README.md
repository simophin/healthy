# Healthy service check

This is a simple service check that can be used to check the health of a service.
It is a simple HTTP server that listens on a port and returns a 200 OK response when the service is healthy.

A service needs to register itself with the service check by sending a PUT request to the service check server,
periodically.

## Usage

### Write API: `PUT /health/:name`

Register a service with the service check. The `:name` parameter is the name of the service.

#### Mandatory header value:

`x-write-token`: The token that the service check uses to authenticate the service. This token must equal to the one
passed in from the command line option (or environment variables).

#### Optional query parameters:

`deadline_seconds`: The deadline for the next health check.
If the service does not send a PUT request to the service check within this time, the service check will consider
the service unhealthy. The default value is 15 seconds.

### Read API: `GET /health/:name`

Check the health of a service. The `:name` parameter is the name of the service.

#### Response:

`200`: The service is healthy.

`410`: The service is gone.

## Environment variables

`TOKEN`: The token that the service check uses to authenticate the service. **Required**.

`LISTEN_ADDR`: The address that the service check listens on. The default value is `127.0.0.1:3400`.
