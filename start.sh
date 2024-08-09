echo "==================================== Deployment Started ===================================="
cd /root/app
echo "===================== 1. Stop and Remove Running Container ====================="
docker rm magic_axum_api --force 2> /dev/null
docker image rm magic_axum_api 2> /dev/null
echo "===================== 2. Load New Image and Start Container ====================="
docker load -i magic_axum_api.tar
docker run --restart=always --name magic_axum_api -p 3000:3000 -d magic_axum_api
echo "==================================== Deployment Finished ===================================="