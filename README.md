# PHP Error Message Format Extension

Specify custom error message formats for PHP.

## Usage

```php
$url = $_SERVER['REQUEST_URI'];
ini_set( 'error_message_format', '{message} from URL' . $url );
```

## Resources

- [ext-php-rs Documentation](https://docs.rs/ext-php-rs)
- [ext-php-rs Guide](https://davidcole1340.github.io/ext-php-rs)
- [PHP Extension Development](https://www.phpinternalsbook.com/)

## License

This project is licensed under the MIT License.
