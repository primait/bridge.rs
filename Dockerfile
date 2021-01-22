FROM 595659439703.dkr.ecr.eu-west-1.amazonaws.com/rust:1.49.0-1

WORKDIR /code

COPY entrypoint /code/entrypoint

RUN chown -R app:app /code

# Serve per avere l'owner dei file scritti dal container uguale all'utente Linux sull'host
USER app

ENTRYPOINT ["./entrypoint"]
