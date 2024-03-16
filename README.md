# Reddit Clone
![Branches](./badges/coverage-branches.svg)
![Functions](./badges/coverage-functions.svg)
![Lines](./badges/coverage-lines.svg)
![Statements](./badges/coverage-statements.svg)
![Jest coverage](./badges/coverage-jest%20coverage.svg)

Development Status: Completed

# Note

The backend in this repository is no longer in development. It is being rewritten in Go and is located in the repository `matttm/reddit-clone-backend`.

## Description

A GraphQL based web application meant to be an immitation of Reddit.

### Discussion

There are some important design decisions I want to discuss.

Firstly is the app's authentication. I initially was going to just use a token, stored in local storage for authentication, but because the app uses NextJS, which allows for server side rendering, I would not aalways have access to local storage. So, for authentication, I delegated the login request from a fetcher to Next's API routes, inorder to store a secure cookie. I also wanted this authentication state to be available globally as it is being used in multiple components, so for this, I made use of React's Context API.

So when logging in ffor instance, a request is made to Nexts 's `/login` route, and that handler gets the tokenn from the GraphQL server and stores it as a cookie for between the UI and the server side code. This way, the cookie is available in the main app's `getInitialProps`, and is used to authenticate the user on every request to the server, for the user's entire time on the site. After authenticating, the status is then stored in React's global context.

Since the GraphQL library that I am using is invoked on the client's machine, the token was also needed in the browser, independent of requests to Next, so the `/login` route also sends the token back to the client. Thien it is stored in local storage and used for subsequent GraphQL requets. This part may be rewritten at some point to make the auth token entirely cookie based.

## Getting Started

First install the desired dependencies by running the following, in the project root
```
npm i
```

After starting the backend as described in its own repo, run the following in another terminal
```
npm run dev
```

## Enhancements

### Migrations

If you need to add a migration, the `migrations` folder must be specified

From the backend folder:

```bash
npx typeorm migration:generate -n migratin-name -d src/migration
```

## Authors

-   Matt Maloney : matttm

## Contribute

If you want to contribute, just send me a message.
