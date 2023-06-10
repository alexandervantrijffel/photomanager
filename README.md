# PhotoManager

Runs a GraphQL server at [http://localhost:8998/graphql](http://localhost:8998/graphql). The server list image files and supports organising files in folders for further processing. 

Use the app SMBSync2 to sync photos from your Android based phone to a samba share so that they can be processed by PhotoManager.

On IOS, use PhotoSync to sync photos to a samba share.

The app is deployed to a Kubernetes Cluster using GitHub Actions, Kustomize and Argo CD.
