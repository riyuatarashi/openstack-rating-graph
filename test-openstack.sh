#!/bin/bash

# Test script to verify OpenStack setup for the Cost Dashboard

echo "🔍 OpenStack Cost Dashboard - Setup Test"
echo "========================================"

# Test 1: Check if OpenStack CLI is installed
echo "📦 Checking OpenStack CLI installation..."
if command -v openstack &> /dev/null; then
    echo "✅ OpenStack CLI is installed: $(openstack --version)"
else
    echo "❌ OpenStack CLI is not installed"
    echo "   Please install it with: pip install python-openstackclient"
    exit 1
fi

# Test 2: Check authentication
echo ""
echo "🔐 Testing OpenStack authentication..."
if openstack token issue &> /dev/null; then
    echo "✅ OpenStack authentication is working"
    echo "   Project: $(openstack token issue -f value -c project_id 2>/dev/null || echo "Unknown")"
else
    echo "❌ OpenStack authentication failed"
    echo "   Please set up your OpenStack credentials:"
    echo "   - Source your OpenStack RC file: source ~/openstack-rc.sh"
    echo "   - Or set environment variables (see README.md)"
    exit 1
fi

# Test 3: Check if rating service is available
echo ""
echo "📊 Testing rating service availability..."
if openstack rating --help &> /dev/null; then
    echo "✅ Rating service plugin is available"
else
    echo "❌ Rating service plugin is not available"
    echo "   Please install it with: pip install python-cloudkittyclient"
    exit 1
fi

# Test 4: Try the actual command
echo ""
echo "🚀 Testing the actual rating command..."
DATE_STRING=$(date +'%Y-%m-01T00:00:00+00:00')
echo "   Command: openstack rating dataframes get -b $DATE_STRING -c Resources -f json"

if openstack rating dataframes get -b "$DATE_STRING" -c Resources -f json &> /dev/null; then
    echo "✅ Rating command executed successfully"
    echo "   The dashboard should work correctly!"
else
    echo "⚠️  Rating command failed"
    echo "   This might be because:"
    echo "   - No rating data available for the current month"
    echo "   - Rating service is not configured"
    echo "   - You don't have permission to access rating data"
    echo "   The dashboard will still run but show no data"
fi

echo ""
echo "🎉 Setup test completed!"
echo "   You can now run: cargo run"
