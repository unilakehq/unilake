// TODO: implement scanning functionality for scanning for tags using regex and LLM
// Idea: new sql statement to scan for tags, returning a list of matching tags
// SCAN TAGS (<QUERY>), we will create a temp table in the backend and scan for tags using regex
// Return a result set with Column, Tag Name
// https://github.com/madisonmay/CommonRegex/blob/master/commonregex.py

/*
Instructions: You are a database schema classifier expert, you shall only respond in json as provided by the schema mentioned below.
Given the below <e> sections, please return all detected PII information in the following json format {"catalog": <name of the catalog>, "table": <name of the table>, "entity": <name of the entity column>, "attribute": <name of the entity attribute>, "pii_type": <type of PII detected, empty if none>}.

The following pii_types are known:
pii_ip,pii_userid,pii_firstname,pii_lastname,pii_email,pii_fullname,pii_postal,pii_phonenumber,pii_country,pii_province,pii_creditcard,pii_bankaccount,pii_unclassified (for all other types of PII)

<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:Title</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:EventTime</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:ClientIP</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:UserID</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:MobilePhone</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:Address</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:Age</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:Income</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:SocialNetwork</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:RemoteIP</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:SkinColor</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:cc_number</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:cc_approved</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:cc_issuer</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:iban</e>
<e>Catalog:customers,Table:customer_details,Entity:customer_events,Attribute:account_holder</e>
 */
