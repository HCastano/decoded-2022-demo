    INK!                                         RUNTIME
  ────────                                      ─────────


SEND SCHEDULE  ─────────────────────────────► BUILD CONTRACT
   MESSAGE                                        CALL

      .                                             │
      .                                             │
      .                                             │
      .                                             ▼
      .
      .                                       BUILD SCHEDULER
      .                                           CALL
      .
      .                                             │
      .                                             │
      .                                             │
                                                    ▼
HANDLE TRIGGER  ◄──────────────────────────── TRIGGER SCHEDULER
   MESSAGE



//////////////////////////////////////////////////////////////////////////////////////////



┌──────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                          │
│                               ┌───────────────────────────┐                              │
│                               │                           │                              │
│                               │                           │                              │
│                               │                           │                              │
│                               │                           │                              │
│                               │                           │                              │
│                               │       INK! CONTRACT       │                              │
│              ┌────────────────┤                           │◄────────────┐                │
│              │                │                           │             │                │
│              │                │                           │             │                │
│              │                │                           │             │                │
│              │                │                           │             │                │
│              │                │                           │             │                │
│              │                └───────────────────────────┘             │                │
│              │                                                          │                │
│              │                                                          │                │
│              │                                                          │                │
│              │                                                          │                │
├──────────────┼──────────────────────────────────────────────────────────┼────────────────┤
│              │                                                          │                │
│              │                                                          │                │
│              │               ┌───────────────────────────┐              │                │
│              │               │         SCHEDULER         │              │                │
│              │               │           CALL            │              │                │
│              │               │                           │              │                │
│              │               │     ┌───────────────┐     │              │                │
│              │               │     │               │     │              │                │
│              │               │     │   CONTRACT    │     │              │                │
│              └─────────────► │     │     CALL      │     │ ─────────────┘                │
│                              │     │               │     │                               │
│                              │     │               │     │                               │
│                              │     └───────────────┘     │                               │
│       RUNTIME                │                           │                               │
│   CHAIN EXTENSION            │                           │                               │
│                              └───────────────────────────┘                               │
│                                                                                          │
└──────────────────────────────────────────────────────────────────────────────────────────┘
