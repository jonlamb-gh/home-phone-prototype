# Display Mockups

|       |                      |
| :---: | :------------------: |
| R0    | Up to 20 characters. |
| R1    | Up to 20 characters. |
| R2    | Up to 20 characters. |
| R3    | Date and time. |

## TODO

`Phone::State == OnGoingCall`

`Phone::State == HandlePendingCall`

`Phone::State == OnGoingCall`

## Idle, `Phone::State == WaitingForEvents`

|       |                      |
| :---: | :------------------: |
| R0    | |
| R1    | |
| R2    | |
| R3    | Thu May 30 01:33 PM |

### Missed Calls

* Only displayed when idle, must ACK/clear to remove

|       |                      |
| :---: | :------------------: |
| R0    | '*' Next &#124; Clear '#' |
| R1    | 245 Missed Calls |
| R2    | (123) 456-7890 |
| R3    | Thu May 30 01:33 PM |

## Call Active

* Number and duration stay display after hangup for N seconds

|       |                      |
| :---: | :------------------: |
| R0    | In/Out: (123) 456-7890 |
| R1    | Duration: 123 sec |
| R2    | |
| R3    | Thu May 30 01:33 PM |

### Incoming Call and Active

* Incoming call should wait N seconds before decline
* Message is displayed for N seconds, until decline

|       |                      |
| :---: | :------------------: |
| R0    | In: (123) 456-7890 |
| R1    | Duration: 123 sec |
| R2    | New: (456) 222-3333 |
| R3    | Thu May 30 01:33 PM |

## Incoming Call

|       |                      |
| :---: | :------------------: |
| R0    | '*' Ans &#124; Decl '#' |
| R1    | Maybe Caller Id |
| R2    | New: (456) 222-3333 |
| R3    | Thu May 30 01:33 PM |
