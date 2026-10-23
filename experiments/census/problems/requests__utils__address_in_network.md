requests.utils.address_in_network (from requests 2.34.2)

Signature, in words: `address_in_network` takes the required arguments `ip`, `net`. It returns a value of type `bool`.

The docstring:

This function allows you to check if an IP belongs to a network subnet

Example: returns True if ip = 192.168.1.1 and net = 192.168.1.0/24
         returns False if ip = 192.168.1.1 and net = 192.168.100.0/24

:rtype: bool
