# Beancount Automation

This project aims to provide a simple API hosted on Vercel that can parse user input strings into Beancount transaction format and automatically add the transaction to your Beancount Github repository. It also provides Telegram bot support so you can directly send message to your bot and get reply from your bot.

The API accepts a POST request with a plain text payload containing the user input string, and returns the Beancount formatted transaction. The transaction is then automatically added to the specified private Beancount Github repository. This can be useful for automating financial transactions and keeping them organized in a private Github repository.

# Usage

To use the API, send a POST request to the API endpoint with a plain text payload containing the user input string, as follows:

```text
2021-09-08 @KFC hamburger 12.40 AUD cba > food
```

The API will return a response with the Beancount formatted transaction, as follows:

```beancount
2021-09-08 * "KFC" "hamburger"
  Expenses:Food          12.40 AUD
  Assets:Bank:CBA
```

The transaction will also be automatically added to the specified private Beancount Github repository.

The whole process can be integrated with Telegram bot, config your bot to send message to the API, and you will get all these things done automaticlaly.

![bot message](https://user-images.githubusercontent.com/1312723/219921978-4fc9e1b7-b2e2-4e48-818f-7964b4a127a7.png)

# Deployment

This project can be deployed on Vercel. To deploy your own instance of the API, follow these steps:

1. Config Repository secrets for Actions with `ORG_ID`,`PROJECT_ID` and `VERCEL_TOKEN`, Github action will deploy the api to Vercel.
2. Once the API is deployed, config environment variables in project settings to have
   - GITHUB_TOKEN, personal access token which has the access to update beancount transactions in your private repo.
   - CONFIG, that's the config for your beancount in toml format, e.g,
     ```toml
     currency = "AUD"
     [accounts]
     amex = "Liabilities:CreditCard:AMEX"
     food = "Expenses:Food"
     car = "Expenses:Car"
     game = "Expenses:Game"
     ```
   * GITHUB_REPO, your beancount private repo, e.g, beancount
   * GITHUB_OWNER, your github account name, e.g, liul85 for me
