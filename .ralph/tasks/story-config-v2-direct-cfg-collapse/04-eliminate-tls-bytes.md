## Task: Eliminate TLS bytes <status>not_started</status> <passes>false</passes>

<description>
Someone previously had this crazy idea to store bytes within the config struct. This causes a crazy amount of issues.
I don't want that at all. I don't want tls bytes to be inside the config struct, nor i ever want them to be written somewhere else.
Instead all components in code use the same tls struct and all use only path based approaches.
Do not store bytes of pem/cert/key whatever in any way inside the code.
Just give the full path to postgres or just give the path to rustls.

Also no more writing the key/cert/pem files to anywhere. Any cert file writing of that kind is forbidden.
</description>

<acceptance_criteria>
- [ ] All versions of storing pem/cert or whatever bytes and/or copying/writing them into new files must be fully and verifiably be gone from code
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly 
</acceptance_criteria>
