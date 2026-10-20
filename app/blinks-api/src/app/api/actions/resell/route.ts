import { NextResponse } from 'next/server';
export function GET() { return NextResponse.json({ message: 'ENTRY resale Action is planned but not enabled.' }, { status: 501, headers: { 'Access-Control-Allow-Origin': '*' } }); }
export function POST() { return NextResponse.json({ message: 'Resale transaction builder is not configured yet.' }, { status: 501, headers: { 'Access-Control-Allow-Origin': '*' } }); }
