#!/bin/bash
# Download missing FX rates from trade-tariff.service.gov.uk
#
# Usage:
#   ./scripts/download-fx-rates.sh              # Download missing rates to bundled folder
#   ./scripts/download-fx-rates.sh ./my-rates   # Download to custom folder
#   ./scripts/download-fx-rates.sh ./my-rates 2018  # Start from specific year
#
# Source: https://www.trade-tariff.service.gov.uk/exchange_rates
# API: https://www.trade-tariff.service.gov.uk/api/v2/exchange_rates/files/monthly_xml_YYYY-MM.xml
#
# Note: The trade-tariff API only provides rates from January 2021 onwards.
# For historical rates (pre-2021), see:
#   https://webarchive.nationalarchives.gov.uk/ukgwa/20231016190054/https://www.gov.uk/government/collections/exchange-rates-for-customs-and-vat
# (Requires browser access due to WAF protection)

set -e

RATES_DIR="${1:-crates/cgt-money/resources/rates}"
START_YEAR="${2:-2021}"
BASE_URL="https://www.trade-tariff.service.gov.uk/api/v2/exchange_rates/files"

# Ensure directory exists
mkdir -p "$RATES_DIR"

# Get current year and month
CURRENT_YEAR=$(date +%Y)
CURRENT_MONTH=$(date +%m)

echo "Downloading FX rates to: $RATES_DIR"
echo "Checking years $START_YEAR to $CURRENT_YEAR..."
echo "(Note: API only has rates from 2021 onwards)"
echo

downloaded=0
skipped=0
failed=0

for year in $(seq "$START_YEAR" "$CURRENT_YEAR"); do
    for month in $(seq -w 1 12); do
        # Skip future months
        if [ "$year" -eq "$CURRENT_YEAR" ] && [ "$month" -gt "$CURRENT_MONTH" ]; then
            continue
        fi

        filename="${year}-${month}.xml"
        filepath="$RATES_DIR/$filename"

        if [ -f "$filepath" ]; then
            skipped=$((skipped + 1))
            continue
        fi

        url="${BASE_URL}/monthly_xml_${year}-${month}.xml"
        echo -n "Downloading $filename... "

        if curl -sf "$url" -o "$filepath" 2>/dev/null; then
            # Verify it's valid XML (not an error page)
            if head -1 "$filepath" | grep -q '<?xml'; then
                echo "OK"
                downloaded=$((downloaded + 1))
            else
                echo "INVALID (not XML)"
                rm -f "$filepath"
                failed=$((failed + 1))
            fi
        else
            echo "FAILED"
            rm -f "$filepath"
            failed=$((failed + 1))
        fi
    done
done

echo
echo "Summary:"
echo "  Downloaded: $downloaded"
echo "  Skipped (existing): $skipped"
echo "  Failed: $failed"
echo
echo "Total files in $RATES_DIR: $(find "$RATES_DIR" -name '*.xml' 2>/dev/null | wc -l | tr -d ' ')"
