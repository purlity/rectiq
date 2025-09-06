Report security issues to security@rectiq.com. Do not open public issues.

## Provenance & Verification
We sign `SHA256SUMS.txt` with minisign for every release. The public key is published at `SECURITY/rectiq-minisign.pub` or can be provided via `RECTIQ_MINISIGN_PUBKEY`.

Verify example:

```bash
minisign -Vm SHA256SUMS.txt -P RWQBneg2bkll5miWVfNNPaCuz3wAjzVYcjwe0A5uR07iKxz24QlpGaj6
```

Installer verification:
- Default: signature + per-asset SHA256 verification enforced.
- Dev-only override: set `RECTIQ_INSECURE_SKIP_VERIFY=1` to bypass verification (not recommended).
